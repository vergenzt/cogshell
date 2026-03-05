from mmap import mmap
import os
import re
import stat
import subprocess as sp
import sys
from collections.abc import Iterator
from contextlib import ExitStack, contextmanager
from dataclasses import asdict, dataclass
from functools import cache, cached_property
from hashlib import md5
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Optional

from .config import LINE_SEPS, CogShellEnviron, Config, MarkerType
from .errors import CogShellError

CHECKSUM_FN = md5


@cache
def line_for(file_content: bytes, loc: int) -> int:
    return file_content.count(b"\n", 0, loc)


@dataclass
class EmbeddedProgram:
    config: Config
    filename: str
    file_content: mmap

    prog_start_marker: re.Match[bytes]
    prog_end_marker: re.Match[bytes]
    output_end_marker: re.Match[bytes]

    @cached_property
    def _program_raw(self) -> bytes:
        "Raw bytes of program between start marker and end marker"
        return self.file_content[
            self.prog_start_marker.end() + 1 : self.prog_end_marker.start()
        ]

    @cached_property
    def _prog_start_line_start(self) -> int:
        "Location of the start of the line containing the prog_start_marker"
        return max(self.file_content.rfind(b"\n", 0, self.prog_start_marker.start()), 0)

    @cached_property
    def _program_lines(self) -> list[bytes]:
        "Lines between prog_start_marker and prog_end_marker"
        return self.file_content[
            self._prog_start_line_start : self.prog_end_marker.start()
        ].splitlines()

    @cached_property
    def _program_lines_common_pfx(self) -> bytes:
        "Common prefix of lines between prog_start_marker and prog_end_marker"
        non_blank_lines = [line for line in self._program_lines if line.strip()]
        return os.path.commonprefix(non_blank_lines)  # type: ignore

    @cached_property
    def program(self) -> bytes:
        """
        Program bytes after stripping common prefix from program lines.

        See:
        - https://github.com/vergenzt/cog/blob/1ef2543fe49f4b322736f37bb9959579e440d56d/cogapp/cogapp.py#L477-L479
        - https://github.com/vergenzt/cog/blob/1ef2543fe49f4b322736f37bb9959579e440d56d/cogapp/cogapp.py#L128-L131
        """
        if b"\n" in self._program_raw:
            return re.sub(
                b"^" + re.escape(self._program_lines_common_pfx),
                b"",
                self._program_raw,
                re.MULTILINE,
            )
        else:
            return self._program_raw

    @cached_property
    def output_prev_raw(self) -> bytes:
        """
        Raw bytes between prog_end_marker and output_end_marker
        """
        return self.file_content[
            self.prog_end_marker.end() + 1 : self.output_end_marker.start()
        ]

    @cached_property
    def _output_end_line_end(self) -> int:
        """
        Location of the end of the line containing output_end_marker (or -1 if no newline after marker).
        """
        return self.file_content.find(b"\n", self.output_end_marker.end())

    @cached_property
    def output_prev_hash_match(self) -> re.Match[bytes]:
        "Returns a match which contains a `hash` group containing checksum hash if present, None otherwise."
        hash_match = re.match(
            rb"( *\(checksum: (?P<hash>[a-f0-9]+)\))?",
            self.file_content[self.output_end_marker.end() : self._output_end_line_end],
        )
        assert hash_match, "Regex should always match (may be empty though)"
        return hash_match

    @property
    def _output_prev_hash_matched(self) -> Optional[bytes]:
        "Returns matched checksum hex digest if present in file, None otherwise."
        return self.output_prev_hash_match.group("hash")

    @cached_property
    def output_prev_hash(self) -> bytes:
        hasher = CHECKSUM_FN()
        hasher.update(self.output_prev_raw)
        return hasher.hexdigest().encode()

    @cached_property
    def output_whitespace_pfx(self) -> bytes:
        """
        Whitespace to prepend to non-blank output lines.

        To be compatible with `cogapp`, we only prefix output with leading common
        whitespace in the *first* and *last* program lines.
        https://github.com/vergenzt/cog/blob/1ef2543fe49f4b322736f37bb9959579e440d56d/cogapp/cogapp.py#L137
        """
        first_and_last_lines = [
            line
            for i, line in enumerate(self._program_lines)
            if i in (0, len(self._program_lines) - 1)
        ]
        leading_whitespaces = [
            line[: -len(line.lstrip())] for line in first_and_last_lines if line.strip()
        ]
        return os.path.commonprefix(leading_whitespaces)  # type: ignore

    @cached_property
    def output_prev(self) -> bytes:
        """
        Previous output bytes after stripping CogShell-injected whitespace.
        """
        return re.sub(
            b"^" + re.escape(self.output_whitespace_pfx),
            b"",
            self.output_prev_raw,
            re.MULTILINE,
        )

    @cached_property
    def execution_dir_name(self) -> str:
        return "-".join(
            [
                "cogshell",
                self.filename,
                str(self.prog_end_marker.start()),
            ]
        )


def get_embedded_programs(
    config: Config, filename: str
) -> Iterator[EmbeddedProgram]:
    """
    Find and yield program instances from then given file with the given config.
    """

    markers = config.markers
    marker_types = config.marker_types_by_marker

    with open(filename, "rb") as file:
        file_mmap = mmap(file.fileno(), 0)
        matches_iter = re.finditer(config.markers_re, file_mmap)

        while True:
            matches: list[re.Match[bytes]] = []

            # try to match each of the markers
            for next_marker_type in MarkerType:
                next_marker = markers[next_marker_type]
                next_match = next(matches_iter, None)
                if not next_match:
                    # no marker found and we're just starting out
                    if not matches:
                        # then stop trying to look for markers
                        break
                    # no marker found and we previously started a marker series
                    else:
                        prev_marker = matches[-1].group()
                        prev_marker_type = marker_types[prev_marker]
                        raise CogShellError(
                            f"Missing {next_marker_type.name} marker `{next_marker!r}` following {prev_marker_type} marker `{prev_marker!r}`",
                            file=str(file),
                            line=line_for(file_mmap, matches[-1].start()),
                        )
                # marker found, but not the right one
                elif (marker := next_match.group()) != marker:
                    marker_type = marker_types[marker]
                    raise CogShellError(
                        f"Unexpected {marker_type} marker `{marker!r}`, expected {next_marker_type} marker `{next_marker!r}`",
                        file=str(file),
                        line=line_for(file_mmap, next_match.start()),
                    )
                # correct marker found
                else:
                    matches.append(next_match)

            yield EmbeddedProgram(config, filename, file_mmap, *matches)


@contextmanager
def execution_dir(prog: EmbeddedProgram) -> Iterator[Path]:
    with TemporaryDirectory(prefix=prog.execution_dir_name + "-") as tmpdir:
        yield Path(tmpdir)


def execution_env(prog: EmbeddedProgram, exec_dir: Path) -> CogShellEnviron:
    env = CogShellEnviron(
        EXEC_DIR=str(exec_dir),
        FILE=os.path.abspath(prog.filename),
        PROGRAM=str(program_file := exec_dir / "program"),
        OUTPUT_PREV=str(output_prev := exec_dir / "output_prev"),
        OUTPUT_NEXT=str(output_next := exec_dir / "output_next"),
    )
    program_file.write_bytes(prog.program)
    program_file.chmod(stat.S_IEXEC | stat.S_IREAD)
    output_prev.write_bytes(prog.output_prev)
    output_next.touch()
    return env


def execute(prog: EmbeddedProgram) -> Iterator[bytes]:
    "Execute the found program."

    with ExitStack() as stack:
        exec_dir = stack.enter_context(execution_dir(prog))

        output_checksum = prog.config.output_checksum
        if prog.config.output_checksum:
            if not (hash_match := prog.output_prev_hash_match):
                print(
                    f"Warning: {prog.filename}({line_for(prog.output_end_marker.end())}): {output_checksum=} but no output checksum detected",
                    file=sys.stderr,
                )
            elif hash_match.group("hash") != prog.output_prev_hash:
                raise CogShellError(
                    "Output checksum does not match!",
                    file=str(prog.filename),
                    line=line_for(hash_match.start("hash")),
                )
            else:
                pass

        exec_env = execution_env(prog, exec_dir)
        exec_env_dict = {
            (
                prog.config.envvar_prefix
                + (asdict(prog.config.envvar_names).get(fieldname) or fieldname.upper())
            ): val
            for fieldname, val in asdict(exec_env).items()
        }

        sp.run(
            prog.program,
            shell=True,
            check=True,
            env=exec_env_dict,
            cwd=exec_dir,
            stdin=(
                stack.enter_context(open(exec_env.OUTPUT_PREV, "rb"))
                if prog.config.prev_output_on_stdin
                else sp.DEVNULL
            ),
            stdout=(
                stack.enter_context(open(exec_env.OUTPUT_NEXT, "wb"))
                if prog.config.next_output_on_stdout
                else None
            ),
        )

        output_fmted = exec_dir / "output_next_fmted"

        with open(prog.filename, "wb") as f:
            f.write(prog.file_content[: prog.prog_end_marker.end()])

            for line_sep in LINE_SEPS:
                if prog.output_prev_raw.startswith(line_sep):
                    f.write(b"\n")
                    break

            hasher = CHECKSUM_FN()

            last_output_line: bytes = b""
            with open(exec_env.OUTPUT_NEXT, "rb") as output:
                while output_line_raw_with_sep := output.readline():
                    line_sep = next(
                        filter(output_line_raw_with_sep.endswith, LINE_SEPS), b""
                    )
                    output_line_raw = output_line_raw_with_sep[: -len(line_sep)]
                    output_line = b"".join(
                        [
                            prog.output_whitespace_pfx,
                            output_line_raw,
                            prog.config.output_line_suffix,
                            line_sep,
                        ]
                    )
                    f.write(output_line)
                    hasher.update(output_line)
                    last_output_line = output_line

            for line_sep in LINE_SEPS:
                if prog.output_prev_raw.endswith(
                    line_sep
                ) and not last_output_line.endswith(line_sep):
                    f.write(line_sep)
                    f.write(prog.output_whitespace_pfx)

            f.write(prog.config.markers[MarkerType.OUTPUT_END])
            if output_checksum:
                f.write(b" (")
                f.write(hasher.hexdigest().encode())
                f.write(b")")

            f.write(prog.file_content[prog.output_prev_hash_match.end() + 1 :])
