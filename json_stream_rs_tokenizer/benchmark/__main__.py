try:
    from .cli import main
except ImportError as e:
    raise ImportError(
        "benchmark dependencies not installed, please consult the README"
    ) from e

if __name__ == "__main__":
    exit(main())
