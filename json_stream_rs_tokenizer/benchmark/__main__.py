try:
    from .cli import main
except ImportError as _e:
    raise ImportError(
        "benchmark dependencies not installed, please consult the README"
    ) from _e

if __name__ == "__main__":
    exit(main())
