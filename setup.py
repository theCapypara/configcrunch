from setuptools import setup, find_packages

setup(
    name='configcrunch',
    version='0.1',
    packages=find_packages(),
    include_package_data=True,
    install_requires=[
        'schema >= 0.6',
        'pyyaml >= 3.13',
        'jinja2 >= 2.10'
    ]
)
