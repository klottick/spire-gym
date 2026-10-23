"""Compare only equivalent work on equivalent builds and runners."""
import argparse
import json
import math
import sys
from decimal import Decimal
from fractions import Fraction
from pathlib import Path
from .results import canonical_bytes, median_ns_per_action, summarize, validate_result

ROW_KEYS = ("version", "contract_sha256", "profile", "character", "ascension", "seed",
            "batch_size", "workers", "input_sha256", "actions_sha256", "accepted_per_episode",
            "expected_contract_version", "endpoint")


def _identity_known(doc):
    values = [*doc["machine"].values(), doc["build"]["rustc_vv"], doc["build"]["target"]]
    return all(isinstance(v, str) and v.strip().lower() not in {"", "unknown", "n/a"} for v in values)


def compare_results(baseline, candidate, *, max_regression_percent=None, strict=False):
    validate_result(baseline); validate_result(candidate)
    if baseline["schema"] != "spire-gym-performance-v1" or candidate["schema"] != "spire-gym-performance-v1":
        raise ValueError("comparison requires final results with build and machine identity")
    budget = None
    if max_regression_percent is not None:
        if type(max_regression_percent) not in {int, float} or not math.isfinite(max_regression_percent) or max_regression_percent < 0:
            raise ValueError("regression budget must be finite and nonnegative")
        budget = Fraction(Decimal(str(max_regression_percent)))
    old = {(c["id"], c["boundary"]): c for c in baseline["cases"]}
    new = {(c["id"], c["boundary"]): c for c in candidate["cases"]}
    common_reasons = [key for key in ("machine", "build", "sampling")
                      if canonical_bytes(baseline[key]) != canonical_bytes(candidate[key])]
    if not _identity_known(baseline) or not _identity_known(candidate):
        common_reasons.append("unknown build/runner identity")
    rows = []; passed = True; comparable = 0
    for key in sorted(old.keys() | new.keys()):
        row = {"id": key[0], "boundary": key[1]}
        if key not in old:
            row["status"] = "added"
        elif key not in new:
            row["status"] = "missing"
            if strict: passed = False
        else:
            a, b = old[key], new[key]
            reasons = common_reasons + [f for f in ROW_KEYS if canonical_bytes(a[f]) != canonical_bytes(b[f])]
            if reasons:
                row.update(status="incompatible", reasons=reasons)
                if strict: passed = False
            else:
                comparable += 1
                old_ns = median_ns_per_action(a["samples"])
                new_ns = median_ns_per_action(b["samples"])
                change = (new_ns / old_ns - 1) * 100
                old_summary, new_summary = summarize(a["samples"]), summarize(b["samples"])
                over_budget = budget is not None and change > budget
                if over_budget: passed = False
                row.update(status="comparable", baseline=old_summary, candidate=new_summary,
                           latency_change_percent=float(change),
                           throughput_change_percent=(new_summary["median_actions_per_second"] /
                                                      old_summary["median_actions_per_second"] - 1) * 100,
                           over_budget=over_budget)
        rows.append(row)
    return {"passed": passed, "comparable": comparable, "rows": rows}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--max-regression-percent", type=float)
    parser.add_argument("--strict", action="store_true")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    try:
        report = compare_results(json.loads(args.baseline.read_text()), json.loads(args.candidate.read_text()),
                                 max_regression_percent=args.max_regression_percent, strict=args.strict)
    except (OSError, ValueError) as error:
        print(f"comparison: {error}", file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps(report, allow_nan=False, indent=2))
    else:
        for row in report["rows"]:
            detail = (f"{row['latency_change_percent']:+.2f}% ns/action; "
                      f"{row['throughput_change_percent']:+.2f}% actions/s" if row["status"] == "comparable"
                      else ", ".join(row.get("reasons", [])))
            print(f"{row['id']} [{row['boundary']}]: {row['status']} {detail}")
        print(f"{report['comparable']} comparable rows; {'PASS' if report['passed'] else 'FAIL'}")
        if not report["comparable"]:
            print("No comparable measurements; this is not a speedup or regression result.")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
