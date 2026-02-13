#!/usr/bin/env python3

# Given a .pgn file containing a single game, attempt to create a series of UCI commands
# for the specified player, which aims to perfectly recreate the game, down to the node.
#
# Bugs, oddities, or just interesting situations, are often seen in PGNs from games played.
# Those games are almost always played with standard Fischer time controls. As a result,
# reproducing those games can be very challenging.
#
# FastChess, and OpenBench's fork of cutechess, supply the node counters most recently
# reported by the playing engine in the PGN. Engines like Torch, which always produce
# a final UCI report at the end of the search, and which have perfect determinism for
# "go nodes <x>", are able to replay games if the exact node counters are known.
#
# This script makes use of a UCI extension, a command called "wait", which will block
# until the current search is done. That allows piping the output of the script, directly
# into an engine as stdin, to set up the engine state.
#
# Suppose that we have a pgn where Torch played an illegal move as white, during an SPRT
# test. We could get a series of commands to reproduce the engine state, up to the final
# search where the illegal move occurred, with the following:
# ./reproduce.py --pgn bug.pgn --white --nodes --option.Hash=16

import argparse

import chess.pgn

import chess
import common

def iterate_uci_options(unknown):
    for arg in unknown:
        if "=" in arg and arg.startswith("--option."):
            yield arg[len("--option.") :].split("=")


def parse_args():

    p = argparse.ArgumentParser(description="Add UCI options with: --option.Name=Value")
    p.add_argument(
        "--pgn", type=str, required=True, help=".pgn with only a single game"
    )

    # Must pick between replicating game with fixed nodes or fixed depth
    p.add_argument(
        "--nodes", action="store_true", help='Generate commands using "go nodes"'
    )
    p.add_argument(
        "--depth", action="store_true", help='Generate commands using "go depth"'
    )

    # Provide commands only for the desired colour
    p.add_argument("--white", action="store_true", help="Generate commands for White")
    p.add_argument("--black", action="store_true", help="Generate commands for Black")

    args, unknown = p.parse_known_args()

    if args.nodes == args.depth:
        raise Exception("Must use either --nodes or --depth")

    if args.white == args.black:
        raise Exception("Must use either --white or --black")

    return args, iterate_uci_options(unknown)


def main():

    args, uci_options = parse_args()

    with open(args.pgn) as pgn_file:
        game = chess.pgn.read_game(pgn_file)

    if not game:
        raise Exception("Empty PGN file")

    print("uci")
    for opt_name, opt_value in uci_options:
        print("setoption name %s value %s" % (opt_name, opt_value))
    print("ucinewgame")
    print("isready")

    fen = game.headers.get("FEN", None)
    pos = "position fen %s moves" % (fen) if fen else "position startpos moves"

    node = game
    while node.variations:
        next_node = node.variation(0)

        if (node.turn() and args.white) or (not node.turn() and args.black):
            print(pos)

            data = common.parse_fastchess_pgn_comment(next_node.comment)

            depth = data['depth']
            nodes = data['nodes']

            print("go depth %s" % (depth) if args.depth else "go nodes %s" % (nodes))
            print("wait")

        pos += " " + next_node.move.uci()
        node = next_node


if __name__ == "__main__":
    main()
