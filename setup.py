from pathlib import Path
from setuptools import setup
from setuptools_rust import Binding, RustExtension

this_directory = Path(__file__).parent
long_description = (this_directory / "README.md").read_text()

setup(
    name="json-stream-rs-tokenizer",
    version="0.4.4",
    rust_extensions=[
        RustExtension(
            "json_stream_rs_tokenizer.json_stream_rs_tokenizer",
            binding=Binding.PyO3,
            optional=True,
            py_limited_api=False,  # required for num_bigint compat.
        )
    ],
    packages=["json_stream_rs_tokenizer"],
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
            "tqdm>=4.64,<5",
            "contexttimer>=0.3,<0.4",
            "si-prefix>=1.2<2",
            "typer>=0.6,<0.7",
        ],
        "test": ["pytest>7.1,<8"],
    },
    classifiers=[
        "Programming Language :: Rust",
        "Programming Language :: Python :: Implementation :: CPython",
        "Programming Language :: Python :: Implementation :: PyPy",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "License :: OSI Approved :: MIT License",
    ],
)
