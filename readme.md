# preach

a simple markdown-to-web-presentation tool

### install

    pip install preach

### use

1. write a presentation in markdown, separating slides with `---`:

    # hello

    ---

    ![](some/image.jpg)

    this is my presentation

2. compile:

    preach compile my_presentation.md

#### options

- include `-w` to auto-recompile when the file or its assets change
- specify an output directory with `-o my_output`
- specify a different stylesheet to use with `-s my_style.css`