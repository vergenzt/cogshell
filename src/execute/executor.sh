#!/usr/bin/env bash
set -euo pipefail

if test -f "${COGSH_PROLOGUE:-}"; then
  echo "executing prologue for $COGSH_SOURCE..." >&2
  source "$COGSH_PROLOGUE" --
fi
printf "\n%s\n" "$1"; shift  # prologue terminator

for ((i=1; i<=$COGSH_NUM_BLOCKS; i++)); do
  ref="COGSH_BLOCK_PROG_$i";              export COGSH_BLOCK_PROG="${!ref}"
  ref="COGSH_BLOCK_LINE_$i";              export COGSH_BLOCK_LINE="${!ref}"
  ref="COGSH_BLOCK_COL_$i";               export COGSH_BLOCK_COL="${!ref}"
  ref="COGSH_BLOCK_OFFSET_$i";            export COGSH_BLOCK_OFFSET="${!ref}"
  ref="COGSH_BLOCK_OUTPUT_LINE_PFX_$i";   export COGSH_BLOCK_OUTPUT_LINE_PFX="${!ref}"
  ref="COGSH_BLOCK_OUTPUT_PREV_$i";       export COGSH_BLOCK_OUTPUT_PREV="${!ref}"

  echo "executing block at $COGSH_SOURCE:$COGSH_BLOCK_LINE:$COGSH_BLOCK_COL..." >&2

  source "$COGSH_BLOCK_PROG" --
  printf "\n%s\n" "$1"; shift  # block terminator
done
