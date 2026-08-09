import re
import subprocess as sp
from pathlib import Path
from sys import argv
from zipfile import ZipFile

wheel_path = argv[1]

sp.run(["cargo", "cyclonedx"], check=True)
[sbom_file_path] = Path().glob("*.cdx.xml")

with ZipFile(sbom_file_path, "a") as zip_file:
    for name in zip_file.namelist():
        if (m := re.match(r".*\.dist-info", name)) is not None:
            dist_info_dir_path = Path(m.group(0))
        zip_file.write(
            sbom_file_path, arcname=str(dist_info_dir_path / sbom_file_path.name)
        )
