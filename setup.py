from setuptools import setup, find_packages

setup(
    name='configcrunch',
    version='0.1',
    packages=find_packages(),
    description='TODO',  # TODO
    long_description='TODO - Project will be available starting May',  # TODO
    install_requires=[
        'schema >= 0.6',
        'pyyaml >= 3.13',
        'jinja2 >= 2.10'
    ],
    # TODO
    classifiers=[
        'Development Status :: 4 - Beta',
        'Programming Language :: Python',
        'Intended Audience :: Developers',
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python :: 3.6',
        'Programming Language :: Python :: 3.7',
    ]
)
