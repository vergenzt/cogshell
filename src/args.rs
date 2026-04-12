use std::{
    fs::File,
    io::{self, Read, stdin},
    path::PathBuf,
};

use bpaf::*;

#[derive(Debug, Clone)]
pub enum PathOrStdin {
    Path(PathBuf),
    Stdin,
}

impl PathOrStdin {
    pub fn open(&self) -> io::Result<Box<dyn Read>> {
        Ok(match self {
            Self::Path(path) => Box::new(File::open(path)?),
            Self::Stdin => Box::new(stdin()),
        })
    }
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Args {
    /// Checksum the output to protect it against accidental change
    checksum: bool,

    files: Vec<PathOrStdin>,
    // args:
    // _parser.add_argument(
    //     "args",
    //     metavar="[INFILE | @FILELIST | &FILELIST]",
    //     nargs=argparse.ZERO_OR_MORE,
    //     help=dedent("""
    //         FILELIST is the name of a text file containing file names or
    //         other @FILELISTs.

    //         For @FILELIST, paths in the file list are relative to the working
    //         directory where cog was called.  For &FILELIST, paths in the file
    //         list are relative to the file list location."
    //     """),
    // )

    // hash_output: bool = False
    // _parser.add_argument(
    //     "-c",
    //     dest="hash_output",
    //     action="store_true",
    //     help="Checksum the output to protect it against accidental change.",
    // )

    // delete_code: bool = False
    // _parser.add_argument(
    //     "-d",
    //     dest="delete_code",
    //     action="store_true",
    //     help="Delete the Python code from the output file.",
    // )

    // defines: dict[str, str] = field(default_factory=dict)
    // _parser.add_argument(
    //     "-D",
    //     dest="defines",
    //     type=_parse_define,
    //     metavar="name=val",
    //     action=_UpdateDictAction,
    //     help="Define a global string available to your Python code.",
    // )

    // warn_empty: bool = False
    // _parser.add_argument(
    //     "-e",
    //     dest="warn_empty",
    //     action="store_true",
    //     help="Warn if a file has no cog code in it.",
    // )

    // include_path: list[str] = field(default_factory=list)
    // _parser.add_argument(
    //     "-I",
    //     dest="include_path",
    //     metavar="PATH",
    //     type=lambda paths: map(os.path.abspath, paths.split(os.path.pathsep)),
    //     action="extend",
    //     help="Add PATH to the list of directories for data files and modules.",
    // )

    // encoding: str = "utf-8"
    // _parser.add_argument(
    //     "-n",
    //     dest="encoding",
    //     metavar="ENCODING",
    //     help="Use ENCODING when reading and writing files.",
    // )

    // output_name: str | None = None
    // _parser.add_argument(
    //     "-o",
    //     dest="output_name",
    //     metavar="OUTNAME",
    //     help="Write the output to OUTNAME.",
    // )

    // prologue: str = ""
    // _parser.add_argument(
    //     "-p",
    //     dest="prologue",
    //     help=dedent("""
    //         Prepend the Python source with PROLOGUE. Useful to insert an import
    //         line. Example: -p "import math"
    //     """),
    // )

    // print_output: bool = False
    // _parser.add_argument(
    //     "-P",
    //     dest="print_output",
    //     action="store_true",
    //     help="Use print() instead of cog.outl() for code output.",
    // )

    // replace: bool = False
    // _parser.add_argument(
    //     "-r",
    //     dest="replace",
    //     action="store_true",
    //     help="Replace the input file with the output.",
    // )

    // suffix: str | None = None
    // _parser.add_argument(
    //     "-s",
    //     dest="suffix",
    //     metavar="STRING",
    //     help="Suffix all generated output lines with STRING.",
    // )

    // newline: str | None = None
    // _parser.add_argument(
    //     "-U",
    //     dest="newline",
    //     action="store_const",
    //     const="\n",
    //     help="Write the output with Unix newlines (only LF line-endings).",
    // )

    // make_writable_cmd: str | None = None
    // _parser.add_argument(
    //     "-w",
    //     dest="make_writable_cmd",
    //     metavar="CMD",
    //     help=dedent("""
    //         Use CMD if the output file needs to be made writable. A %%s in the CMD
    //         will be filled with the filename.
    //     """),
    // )

    // no_generate: bool = False
    // _parser.add_argument(
    //     "-x",
    //     dest="no_generate",
    //     action="store_true",
    //     help="Excise all the generated output without running the Pythons.",
    // )

    // eof_can_be_end: bool = False
    // _parser.add_argument(
    //     "-z",
    //     dest="eof_can_be_end",
    //     action="store_true",
    //     help="The end-output marker can be omitted, and is assumed at eof.",
    // )

    // show_version: bool = False
    // _parser.add_argument(
    //     "-v",
    //     dest="show_version",
    //     action="store_true",
    //     help="Print the version of cog and exit.",
    // )

    // check: bool = False
    // _parser.add_argument(
    //     "--check",
    //     action="store_true",
    //     help="Check that the files would not change if run again.",
    // )

    // check_fail_msg: str | None = None
    // _parser.add_argument(
    //     "--check-fail-msg",
    //     metavar="MSG",
    //     help="If --check fails, include MSG in the output to help devs understand how to run cog in your project.",
    // )

    // diff: bool = False
    // _parser.add_argument(
    //     "--diff",
    //     action="store_true",
    //     help="With --check, show a diff of what failed the check.",
    // )

    // markers: Markers = Markers("[[[cog", "]]]", "[[[end]]]")
    // _parser.add_argument(
    //     "--markers",
    //     metavar="'START END END-OUTPUT'",
    //     type=Markers.from_arg,
    //     help=dedent("""
    //         The patterns surrounding cog inline instructions. Should include three
    //         values separated by spaces, the start, end, and end-output markers.
    //         Defaults to '[[[cog ]]] [[[end]]]'.
    //     """),
    // )
}
