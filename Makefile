PROJECT_NAME := bitseed
SRC_DIR := src

DOCKER_USERNAME := bitseed
DOCKER_IMAGE_NAME := $(DOCKER_USERNAME)/$(PROJECT_NAME)
DOCKER_IMAGE_TAG := 0.1.5-debug1

# Target for building the project
build:
	cargo build

# Target for running unit tests
unit-test:
	RUST_LOG=info RUST_BACKTRACE=full cargo test --lib

# Target for running integration tests
integration-test:
	RUST_LOG=info RUST_BACKTRACE=full cargo test --test '*'

# Target for running all tests (unit and integration)
test: unit-test integration-test

# Target for cleaning the project
clean:
	cargo clean

# Target for building the Docker image
docker-build:
	docker build -t $(DOCKER_IMAGE_NAME):$(DOCKER_IMAGE_TAG) .
	docker tag $(DOCKER_IMAGE_NAME):$(DOCKER_IMAGE_TAG) $(DOCKER_IMAGE_NAME):latest

# Target for running the Docker container
docker-run:
	docker run --rm $(DOCKER_IMAGE_NAME):$(DOCKER_IMAGE_TAG)

# Target for pushing the Docker image to Docker Hub
docker-push:
	docker push $(DOCKER_IMAGE_NAME):$(DOCKER_IMAGE_TAG)
	docker push $(DOCKER_IMAGE_NAME):latest

# Target for building and pushing the Docker image
docker-build-and-push: docker-build docker-push

# Default target
.PHONY: default
default: build