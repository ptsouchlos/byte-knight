def parse_fastchess_pgn_comment(comment: str):
    """
    Parses the "comment" section of a PGN node assuming the fastchess format.
    """
    parts = comment.replace(",", " ").split()
    results = {}
    for prt in parts:
        if "/" in prt:
            score, depth = prt.split("/")
            results["score"] = score
            results["depth"] = depth
        elif prt.strip().endswith("s"):
            results["time"] = prt.replace("s", "")
        elif prt.strip().startswith("n="):
            _, nodes = prt.split("=")
            results["nodes"] = nodes
        elif prt.strip().startswith("sd"):
            _, sel_depth = prt.split("=")
            results["sel_depth"] = sel_depth
    return results
