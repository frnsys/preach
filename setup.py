from setuptools import setup

setup(
    name='preach',
    version='0.0.1',
    description='simple presentation builder',
    url='https://github.com/frnsys/preach',
    author='Francis Tseng',
    author_email='f@frnsys.com',
    license='GPLv3',

    packages=['preach'],
    install_requires=[
        'click',
        'jinja2',
        'gfm',
        'python-daemon',
        'watchdog',
        'py-gfm'
    ],
    entry_points='''
        [console_scripts]
        preach=preach:cli
    ''',
)