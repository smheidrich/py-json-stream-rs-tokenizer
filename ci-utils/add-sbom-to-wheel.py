import re
import subprocess as sp
from os import environ
from pathlib import Path
from sys import argv
from zipfile import ZipFile

wheel_path = argv[1]
if len(argv) >= 3:  # local testing
    sbom_file_path = Path(argv[2])
else:  # GitHub Actions CI
    [sbom_file_path] = Path(environ["PROJ_DIR"]).glob("*.cdx.xml")

with ZipFile(wheel_path, "a") as zip_file:
    for name in zip_file.namelist():
        if (m := re.match(r".*\.dist-info", name)) is not None:
            dist_info_dir_path = Path(m.group(0))
            break
    zip_file.write(
        sbom_file_path, arcname=str(dist_info_dir_path / "sboms" / sbom_file_path.name)
    )
