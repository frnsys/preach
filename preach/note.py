import os
import re

md_img_re = re.compile(r'!\[.*?\]\(`?([^`\(\)]+)`?\)')


class Note():
    def __init__(self, path):
        if not os.path.isabs(path):
            path = os.path.join(os.cwd(), path)
        self.path = path
        _, self.filename = os.path.split(self.path)
        self.title, self.ext = os.path.splitext(self.filename)
        self.content = open(path, 'r').read()
        self.dir = os.path.dirname(self.path)

    @property
    def images(self):
        return md_img_re.findall(self.content)
