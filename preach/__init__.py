import os
import click
from functools import partial
from .compile import compile_note
from .watch import watch_note
from .note import Note


@click.group()
def cli():
    pass


@cli.command()
@click.argument('note')
@click.option('-o', '--outdir', help='output directory', default='_build')
@click.option('-w', '--watch', is_flag=True, help='watch for changes')
@click.option('-s', '--style', help='stylesheet to use', default='default')
def compile(note, outdir, watch, style):
    n = Note(os.path.join(os.getcwd(), note))
    f = partial(compile_note, outdir=outdir, stylesheet=style)
    watch_note(n, f) if watch else f(n)
