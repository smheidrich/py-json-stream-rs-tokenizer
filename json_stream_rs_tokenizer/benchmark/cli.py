from sys import stderr

import typer
from si_prefix import si_parse

from . import app


cli = typer.Typer()


@cli.command()
def run(
    json_bytes: str = typer.Option(
        default="2 M",
        metavar="SI_NUMBER",
        help=(
            "Size of random JSON to generate in bytes. "
            "Use of SI prefixes (e.g. 1k = 1000) is possible."
        ),
    )
):
    try:
        json_bytes = si_parse(json_bytes)
    except Exception:
        print(
            f"Could not parse as SI-prefixed number of bytes: '{json_bytes}'",
            file=stderr,
        )
        raise typer.Exit(code=1)
    app.main(json_bytes)


def main():
    return cli()
