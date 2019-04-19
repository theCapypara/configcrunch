pipeline {
    agent any
    options {
        disableConcurrentBuilds()
    }

    stages {

        stage("Prepare Venv") {
            agent {
                docker { image 'python:3.7' }
            }
            steps {
                sh "rm -rf .venv || true"
                sh "virtualenv .venv"
            }
        }

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

        stage('Build') {
            agent {
                docker { image 'python:3.7' }
            }
            steps {
                sh "rm -rf dist build || true"
                sh "source .venv/bin/activate && pip install -r requirements.txt"
                sh "source .venv/bin/activate && python setup.py bdist_wheel"
            }
            post {
                always {
                    archiveArtifacts allowEmptyArchive: true, artifacts: 'dist/*whl', fingerprint: true
                }
            }
        }

        stage('Deploy to PyPI') {
            when {
                branch "release"
            }
            environment {
                TWINE    = credentials('parakoopa-twine-username-password')
            }
            agent {
                docker { image 'python:3.7' }
            }
            steps {
                sh "source .venv/bin/activate && twine -u $TWINE_USR -p $TWINE_PSW upload dist/*"
            }
        }

    }

}