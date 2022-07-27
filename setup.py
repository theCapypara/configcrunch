from setuptools import setup, find_packages
from setuptools_rust import Binding, RustExtension

# README read-in
from os import path
this_directory = path.abspath(path.dirname(__file__))
with open(path.join(this_directory, 'README.rst'), encoding='utf-8') as f:
    long_description = f.read()
# END README read-in

setup(
    name='configcrunch',
    version='1.0.2',
    packages=["configcrunch"],
    package_data={"configcrunch": ["py.typed", "*.pyi"]},
    rust_extensions=[RustExtension(f"configcrunch._main", binding=Binding.PyO3)],
    description='Configuration parser based on YAML-Files with support for variables, overlaying and hierarchies',
    long_description=long_description,
    long_description_content_type='text/x-rst',
    url='https://github.com/theCapypara/configcrunch/',
    install_requires=[
        'schema >= 0.7'
    ],
    classifiers=[
        'Development Status :: 4 - Beta',
        'Programming Language :: Python',
        'Programming Language :: Rust',
        'Intended Audience :: Developers',
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python :: 3.7',
        'Programming Language :: Python :: 3.8',
        'Programming Language :: Python :: 3.9',
        'Programming Language :: Python :: 3.10',
    ],
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
    include_package_data=True,
)
