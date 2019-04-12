import os
# The only supported way to run doctests is when source is working directory.
os.chdir(os.path.join(os.path.dirname(os.path.realpath(__file__)), 'source'))
os.system('python -m sphinx -b doctest . ../build')
