from pathlib import Path
from shutil import move
import subprocess as sp
from sys import argv
from tempfile import TemporaryDirectory

SUFFIX = "dummy4"


def unhyphenate(s: str):
    return s.replace("-", "_")


def get_one(l: list):
    assert len(l) == 1
    return l[0]


if __name__ == "__main__":
    wheel_archive_path = Path(argv[1])
    with TemporaryDirectory() as temp_dir:
        temp_dir_path = Path(temp_dir)

        # unpack wheel to tempdir
        sp.run(
            ["python3", "-m", "wheel", "unpack", wheel_archive_path.absolute()],
            cwd=temp_dir_path,
        )
        extracted_dirs = [x for x in temp_dir_path.iterdir() if x.is_dir()]
        main_dir = get_one(extracted_dirs)

        # get dist-info dir
        dist_info_dir = get_one(
            [x for x in main_dir.iterdir() if x.name.endswith(".dist-info")]
        )

        # process METADATA (equiv. of PKG-INFO in sdists)
        # TODO find out if there is a decent module for parsing this
        pkg_info_lines = (
            (dist_info_dir / "METADATA").read_text().splitlines(keepends=True)
        )
        for i, line in enumerate(pkg_info_lines):
            if line.startswith("Name"):
                project_name = line.split()[1]
                dummy_project_name = project_name + f"-{SUFFIX}"
                # TODO do this properly, not with replace()
                pkg_info_lines[i] = line.replace(
                    project_name, dummy_project_name
                )
                break
        with (dist_info_dir / "METADATA").open("w") as f:
            f.writelines(pkg_info_lines)

        # rename package dir
        package_name = unhyphenate(project_name)
        dummy_package_name = package_name + f"_{SUFFIX}"
        move(main_dir / package_name, main_dir / dummy_package_name)

        # rename dist-info dir
        move(
            dist_info_dir,
            dist_info_dir.with_name(
                # TODO do this properly, not with replace()
                dist_info_dir.name.replace(package_name, dummy_package_name)
            ),
        )

        # rename main dir
        # TODO use "archive name" or sth here instead, != package_name
        dummy_main_dir = main_dir.with_name(
            # TODO do this properly, not with replace()
            main_dir.name.replace(package_name, dummy_package_name)
        )
        move(main_dir, dummy_main_dir)

        # repack as wheel
        # TODO properly, not with replace()
        dummy_wheel_archive_path = temp_dir_path / (
            wheel_archive_path.name.replace(package_name, dummy_package_name)
        )
        sp.run(
            ["python3", "-m", "wheel", "pack", dummy_main_dir],
            cwd=temp_dir_path,
        )

        # move next to original wheel
        move(
            dummy_wheel_archive_path,
            wheel_archive_path.parent / dummy_wheel_archive_path.name,
        )
