#!/usr/bin/env bash
set -euo pipefail

if test -f "${COGSH_PROLOGUE:-}"; then
  echo "executing prologue for $COGSH_SOURCE..." >&2
  source "$COGSH_PROLOGUE"
fi

for ((i=1; i<=$COGSH_NUM_BLOCKS; i++)); do
  export COGSH_BLOCK_PROG=${!COGSH_BLOCK_PROG_$i}
  export COGSH_BLOCK_LINE=${!COGSH_BLOCK_LINE_$i}
  export COGSH_BLOCK_COL=${!COGSH_BLOCK_COL_$i}
  export COGSH_BLOCK_OFFSET=${!COGSH_BLOCK_OFFSET_$i}
  export COGSH_BLOCK_OUTPUT_PFX=${!COGSH_BLOCK_OUTPUT_PFX_$i}
  export COGSH_BLOCK_OUTPUT_PREV=${!COGSH_BLOCK_OUTPUT_PREV_$i}

  echo "executing block at $COGSH_SOURCE:$COGSH_BLOCK_LINE:$COGSH_BLOCK_COL..." >&2

  source "$COGSH_BLOCK_PROG"
done
