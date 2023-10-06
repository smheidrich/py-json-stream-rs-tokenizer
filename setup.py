from pathlib import Path
from setuptools import setup
from setuptools_rust import Binding, RustExtension

this_directory = Path(__file__).parent
long_description = (this_directory / "README.md").read_text()

setup(
    name="json-stream-rs-tokenizer",
    version="0.4.25",
    rust_extensions=[
        RustExtension(
            "json_stream_rs_tokenizer.json_stream_rs_tokenizer",
            binding=Binding.PyO3,
            optional=False,  # is set to True prior to publishing as sdist
            py_limited_api=False,  # required for num_bigint compat.
            debug=False,  # pointless even in develop mode
        )
    ],
    packages=["json_stream_rs_tokenizer", "json_stream_rs_tokenizer.benchmark"],
    zip_safe=False,
    description="A faster tokenizer for the json-stream Python library",
    readme="README.md",
    long_description=long_description,
    long_description_content_type="text/markdown",
    license_files=["LICENSE"],
    project_urls={
        "Repository": (
            "https://github.com/smheidrich/py-json-stream-rs-tokenizer"
        )
    },
    python_requires=">=3.7,<4",
    install_requires=[],
    extras_require={
        "benchmark": [
            "json-stream-to-standard-types>=0.1,<0.2",
            "tqdm>=4.64,<5",
            "contexttimer>=0.3,<0.4",
            "si-prefix>=1.2,<2",
            "typer>=0.6,<0.7",
        ],
        "test": [
            "pytest>7.1,<8",
            "json-stream-rs-tokenizer[benchmark]",
            "json-stream==2.3.2",
        ],
    },
    classifiers=[
        "Programming Language :: Rust",
        "Programming Language :: Python :: Implementation :: CPython",
        "Programming Language :: Python :: Implementation :: PyPy",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "License :: OSI Approved :: MIT License",
    ],
)
