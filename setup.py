from setuptools import setup, find_packages

# README read-in
from os import path
this_directory = path.abspath(path.dirname(__file__))
with open(path.join(this_directory, 'README.rst'), encoding='utf-8') as f:
    long_description = f.read()
# END README read-in

setup(
    name='configcrunch',
    version='0.1.1',
    packages=find_packages(),
    description='Configuration parser based on YAML-Files with support for variables, overlaying and hierarchies',
    long_description=long_description,
    long_description_content_type='text/x-rst',
    url='https://github.com/Parakoopa/configcrunch/',
    install_requires=[
        'schema >= 0.6',
        'pyyaml >= 5.1',
        'jinja2 >= 2.10.1'
    ],
    classifiers=[
        'Development Status :: 3 - Alpha',
        'Programming Language :: Python',
        'Intended Audience :: Developers',
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python :: 3.6',
        'Programming Language :: Python :: 3.7',
    ]
)
