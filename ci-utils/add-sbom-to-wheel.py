import re
import subprocess as sp
from os import environ
from pathlib import Path
from shutil import copy, rmtree
from sys import argv
from zipfile import ZipFile

wheel_path = Path(argv[1])
if len(argv) >= 3:  # local testing
    sbom_file_path = Path(argv[2])
else:  # GitHub Actions CI
    [sbom_file_path] = Path(environ["PROJ_DIR"]).glob("*.cdx.xml")

temp_dir_path = Path(
    environ.get(
        "RUNNER_TEMP",  # GitHub Actions CI
        "/tmp",  # local testing
    )
)
wheel_unpack_path = temp_dir_path / "wheel-unpack"
wheel_unpack_path.mkdir()

sp.run(["wheel", "unpack", "-d", wheel_unpack_path, wheel_path], check=True)
[unpacked_wheel_path] = wheel_unpack_path.iterdir()

[dist_info_dir_path] = unpacked_wheel_path.glob("*.dist-info")
sboms_dir_path = dist_info_dir_path / "sboms"
sboms_dir_path.mkdir()
copy(sbom_file_path, sboms_dir_path)

sp.run(["wheel", "pack", "-d", wheel_path.parent, unpacked_wheel_path], check=True)
[unpacked_wheel_path] = wheel_unpack_path.iterdir()

rmtree(wheel_unpack_path)
