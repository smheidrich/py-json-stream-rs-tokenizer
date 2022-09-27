from pathlib import Path
from shutil import move, rmtree
from sys import argv
import tarfile
from tempfile import TemporaryDirectory
import toml

SUFFIX = "dummy4"


def unhyphenate(s: str):
    return s.replace("-", "_")


if __name__ == "__main__":
    sdist_archive_path = Path(argv[1])
    tar_file = tarfile.open(sdist_archive_path, "r:gz")
    with TemporaryDirectory() as temp_dir:
        temp_dir_path = Path(temp_dir)

        # extract sdist archive to tempdir
        tar_file.extractall(temp_dir_path)
        extracted_dirs = [x for x in temp_dir_path.iterdir() if x.is_dir()]
        assert len(extracted_dirs) == 1
        main_dir = extracted_dirs[0]

        # delete superfluous files
        files_to_keep = ["LICENSE", "PKG-INFO", "pyproject.toml", "README.md"]
        file_paths_to_keep = [main_dir / filename for filename in files_to_keep]
        for path in main_dir.iterdir():
            if path in file_paths_to_keep:
                continue
            if path.is_file():
                path.unlink()
            elif path.is_dir():
                rmtree(path)

        # process pyproject.toml
        with (main_dir / "pyproject.toml").open() as f:
            pyproject_d = toml.load(f)
        # build system
        pyproject_d["build-system"]["build-backend"] = "setuptools.build_meta"
        pyproject_d["build-system"]["requires"] = ["setuptools"]
        # project
        project_name = pyproject_d["project"]["name"]
        dummy_project_name = project_name + f"-{SUFFIX}"
        pyproject_d["project"]["name"] = dummy_project_name
        del pyproject_d["project"]["dependencies"]
        del pyproject_d["project"]["optional-dependencies"]
        # packages
        package_name = unhyphenate(project_name)
        dummy_package_name = package_name + f"_{SUFFIX}"
        pyproject_d["tool"] = {"setuptools": {"packages": [dummy_package_name]}}
        with (main_dir / "pyproject.toml").open("w") as f:
            toml.dump(pyproject_d, f)

        # process PKG-INFO
        # TODO find out if there is a decent module for parsing this
        pkg_info_lines = (
            (main_dir / "PKG-INFO").read_text().splitlines(keepends=True)
        )
        for i, line in enumerate(pkg_info_lines):
            if line.startswith("Name"):
                # TODO do this properly, not with replace()
                pkg_info_lines[i] = line.replace(
                    project_name, dummy_project_name
                )
                break
        with (main_dir / "PKG-INFO").open("w") as f:
            f.writelines(pkg_info_lines)

        # create dummy package dir
        dummy_package_dir = main_dir / dummy_package_name
        dummy_package_dir.mkdir()
        (dummy_package_dir / "__init__.py").touch()

        # rename main dir
        # TODO use "archive name" or sth here instead, != package_name
        dummy_main_dir = main_dir.with_name(
            # TODO do this properly, not with replace()
            main_dir.name.replace(package_name, dummy_package_name)
        )
        move(main_dir, dummy_main_dir)

        # re-archive/compress it
        dummy_archive_path = dummy_main_dir.with_name(
            dummy_main_dir.name + ".tar.gz"
        )
        with tarfile.open(dummy_archive_path, "w:gz") as dummy_tar_file:
            dummy_tar_file.add(dummy_main_dir, arcname=dummy_main_dir.name)

        # move next to original sdist
        move(
            dummy_archive_path,
            sdist_archive_path.parent / dummy_archive_path.name,
        )
