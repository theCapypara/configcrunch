#!/usr/bin/bash
#
## RUN TESTS (Linux only)
#
# This will run all tests in this project.
# The tests are run on Python versions 3.6 - 3.7.
# See tox.ini for details :)
#
# For these tests to run, you need to have Docker installed. The tests will use a Docker image found in
# the Riptide Lib repository.
#
# FOR MAC AND WINDOWS TESTS:
#   Run the commands in the tox.ini on their own (after installing everything).
#   Testing multiple Python versions not supported on these platforms.
#
# If you have problems, try to delete the .tox directory.
#
# This script is not used in CI, see Jenkinsfile instead.
#

# 1. Build the runner image...
mkdir -p /tmp/riptide-docker-tox
curl "https://raw.githubusercontent.com/Parakoopa/riptide-lib/master/test_assets/riptide-docker-tox/Dockerfile" > /tmp/riptide-docker-tox/Dockerfile
curl "https://raw.githubusercontent.com/Parakoopa/riptide-lib/master/test_assets/riptide-docker-tox/entrypoint.sh" > /tmp/riptide-docker-tox/entrypoint.sh
chmod +x /tmp/riptide-docker-tox/entrypoint.sh
docker build -t riptide_docker_tox /tmp/riptide-docker-tox


# 2. Run the image...
docker run \
    -v /var/run/docker.sock:/var/run/docker.sock \
    -e USER=$(id -u) \
    -e DOCKER_GROUP=$(cut -d: -f3 < <(getent group docker)) \
    -v $SSH_AUTH_SOCK:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
    -v $HOME/.ssh:/home/riptide/.ssh:ro \
    -v "/tmp:/tmp" \
    -v "$(pwd):$(pwd)" \
    --network host \
    --workdir $(pwd) \
    riptide_docker_tox \
    "tox"