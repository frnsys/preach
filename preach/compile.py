import os
import shutil
from jinja2 import FileSystemLoader, environment
import re
import markdown
from markdown.inlinepatterns import SimpleTagPattern, ImagePattern
from markdown.util import etree
from mdx_gfm import GithubFlavoredMarkdownExtension as GFM


dir = os.path.dirname(os.path.abspath(__file__))
templ_dir = os.path.join(dir, 'templates')
env = environment.Environment()
env.loader = FileSystemLoader(templ_dir)


def compile_note(note, outdir, stylesheet=None, templ='default'):
    content = note.content
    templ = env.get_template('{}.html'.format(templ))

    # create output directory if necessary
    if not os.path.exists(outdir):
        os.makedirs(outdir)

    # create assets directory if necessary
    assetsdir = os.path.join(outdir, 'assets')
    if not os.path.exists(assetsdir):
        os.makedirs(assetsdir)

    # copy over any local images
    for img_path in note.images:
        # assume http indicates remote image
        if img_path.startswith('http'):
            continue

        # copy img to compiled assets directory
        _, img_name = os.path.split(img_path)
        to_img_path = os.path.join(assetsdir, img_name)
        shutil.copy(img_path, to_img_path)

        # update references to that image in the note
        to_img_path_rel = os.path.relpath(to_img_path, outdir)
        content = content.replace(img_path, to_img_path_rel)

    # default styles
    styles = open(os.path.join(templ_dir, 'style.css'), 'r').read()

    # if a stylesheet was specified, copy it over
    if stylesheet is not None:
        styles = '\n'.join([styles, open(stylesheet, 'r').read()])

    # write the stylesheet
    with open(os.path.join(outdir, 'style.css'), 'w') as f:
        f.write(styles)

    # render the presentation
    html = compile_markdown(content)
    content = templ.render(html=html)

    # save it
    with open(os.path.join(outdir, note.title) + '.html', 'w') as out:
        out.write(content)


def compile_markdown(md):
    """
    Compiles markdown to html.
    """
    return markdown.markdown(md, extensions=[
        GFM(),
        NomadicMD(),
        'markdown.extensions.footnotes',
        MathJaxExtension(),
        FigureCaptionExtension()
    ], lazy_ol=False)


class PDFPattern(ImagePattern):
    def handleMatch(self, m):
        src = m.group(3)
        fig = etree.Element('figure')

        obj = etree.SubElement(fig, 'iframe')
        obj.set('src', src)

        a = etree.SubElement(fig, 'a')
        a.set('href', src)
        a.text = m.group(2) or src.split('/')[-1]

        return fig


class NomadicMD(markdown.Extension):
    """
    An extension that supports:
    - highlighting with the <mark> tag.
    - pdf embedding with the <iframe> tag.
    """
    HIGHLIGHT_RE = r'(={2})(.+?)(={2})' # ==highlight==
    PDF_RE = r'\!\[([^\[\]]*)\]\(`?(?:<.*>)?([^`\(\)]+pdf)(?:</.*>)?`?\)' # ![...](path/to/something.pdf)

    def extendMarkdown(self, md, md_globals):
        highlight_pattern = SimpleTagPattern(self.HIGHLIGHT_RE, 'mark')
        md.inlinePatterns.add('highlight', highlight_pattern, '_end')

        pdf_pattern = PDFPattern(self.PDF_RE)
        md.inlinePatterns.add('pdf_link', pdf_pattern, '_begin')



"""
The below is from: <https://github.com/jdittrich/figureAltCaption>
(Not provided as a pypi package, so reproduced here)

Generates a Caption for Figures for each Image which stands alone in a paragraph,
similar to pandoc#s handling of images/figures

--------------------------------------------

Licensed under the GPL 2 (see LICENSE.md)

Copyright 2015 - Jan Dittrich by
building upon the markdown-figures Plugin by
Copyright 2013 - [Helder Correia](http://heldercorreia.com) (GPL2)
"""
from markdown.blockprocessors import BlockProcessor
from markdown.inlinepatterns import IMAGE_LINK_RE, IMAGE_REFERENCE_RE
FIGURES = [u'^\s*'+IMAGE_LINK_RE+u'\s*$', u'^\s*'+IMAGE_REFERENCE_RE+u'\s*$'] #is: linestart, any whitespace (even none), image, any whitespace (even none), line ends.


# This is the core part of the extension
class FigureCaptionProcessor(BlockProcessor):
    FIGURES_RE = re.compile('|'.join(f for f in FIGURES))
    def test(self, parent, block): # is the block relevant
        # Wenn es ein Bild gibt und das Bild alleine im paragraph ist, und das Bild nicht schon einen figure parent hat, returne True
        isImage = bool(self.FIGURES_RE.search(block))
        isOnlyOneLine = (len(block.splitlines())== 1)
        isInFigure = (parent.tag == 'figure')

        # print(block, isImage, isOnlyOneLine, isInFigure, "T,T,F")
        if (isImage and isOnlyOneLine and not isInFigure):
            return True
        else:
            return False

    def run(self, parent, blocks): # how to process the block?
        raw_block = blocks.pop(0)
        captionText = self.FIGURES_RE.search(raw_block).group(1)

        # create figure
        figure = etree.SubElement(parent, 'figure')

        # render image in figure
        figure.text = raw_block

        # create caption
        figcaptionElem = etree.SubElement(figure,'figcaption')
        figcaptionElem.text = captionText #no clue why the text itself turns out as html again and not raw. Anyhow, it suits me, the blockparsers annoyingly wrapped everything into <p>.


class FigureCaptionExtension(markdown.Extension):
    def extendMarkdown(self, md, md_globals):
        """ Add an instance of FigcaptionProcessor to BlockParser. """
        md.parser.blockprocessors.add('figureAltcaption',
                                      FigureCaptionProcessor(md.parser),
                                      '<ulist')



"""
From <https://github.com/mayoff/python-markdown-mathjax>
"""
class MathJaxPattern(markdown.inlinepatterns.Pattern):
    def __init__(self):
        markdown.inlinepatterns.Pattern.__init__(self, r'(?<!\\)(\$\$?)(.+?)\2')

    def handleMatch(self, m):
        node = markdown.util.etree.Element('mathjax')
        node.text = markdown.util.AtomicString(m.group(2) + m.group(3) + m.group(2))
        return node

class MathJaxExtension(markdown.Extension):
    def extendMarkdown(self, md, md_globals):
        # Needs to come before escape matching because \ is pretty important in LaTeX
        md.inlinePatterns.add('mathjax', MathJaxPattern(), '<escape')