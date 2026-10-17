"""Disposable test-only comparison worker, never a gameplay/runtime entry point."""
import json
from pathlib import Path
import sys

from rust_diff_support import _run_python_case


def main():
    assert sys.get_int_max_str_digits()==0, "comparison child needs unlimited exact decimal conversion"
    request,response=(Path(p) for p in sys.argv[1:])
    assert request.is_absolute() and response.is_absolute()
    case=json.loads(request.read_text())
    result=_run_python_case(case)
    with response.open("x",encoding="utf-8") as output:
        json.dump(result,output,ensure_ascii=True,indent=2)
        output.write("\n")
    print("comparison child int_max_str_digits=0")


if __name__=="__main__":
    main()
