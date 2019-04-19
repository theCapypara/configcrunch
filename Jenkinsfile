pipeline {
    agent any
    options {
        disableConcurrentBuilds()
    }

    stages {

        stage("Build Images") {
            // Builds images that are required for tests
            steps {
                dir("__riptide_lib") {
                    git url: 'https://github.com/Parakoopa/riptide-lib.git'
                    sh "docker build -t riptide_docker_tox test_assets/riptide-docker-tox"
                }
            }
        }

        stage("Tests") {
            steps {
                sh '''#!/bin/bash
                    docker run \
                        -v /var/run/docker.sock:/var/run/docker.sock \
                        -e USER=$(id -u) \
                        -e DOCKER_GROUP=$(cut -d: -f3 < <(getent group docker)) \
                        -v "/tmp:/tmp" \
                        -v "$(pwd):$(pwd)" \
                        --network host \
                        --workdir $(pwd) \
                        riptide_docker_tox \
                        tox
                '''
            }
        }

        stage('Build and Deploy to PyPI') {
            when {
                branch "release"
            }
            agent {
                docker { image 'python:3.7' }
            }
            steps {
                sh pip install -r requirements.txt
                sh python setup.py bdist_wheel
                sh twine upload dist/*
            }
            post {
                always {
                    archiveArtifacts allowEmptyArchive: true, artifacts: 'dist/*whl', fingerprint: true)
                    sh rm -rf dist build || true
                }
            }
        }

    }

}