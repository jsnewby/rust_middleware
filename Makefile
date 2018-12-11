# build output folder
OUTPUTFOLDER = target
# docker image
DOCKER_REGISTRY = 166568770115.dkr.ecr.eu-central-1.amazonaws.com/aeternity
DOCKER_IMAGE = aepp-middleware
DOCKER_TAG = $(shell git describe --always --tags)
# epoch url used at build time
VUE_APP_EPOCH_URL=//sdk-testnet.aepps.com/


.PHONY: list
list:
	@$(MAKE) -pRrq -f $(lastword $(MAKEFILE_LIST)) : 2>/dev/null | awk -v RS= -F: '/^# File/,/^# Finished Make data base/ {if ($$1 !~ "^[#.]") {print $$1}}' | sort | egrep -v -e '^[^[:alnum:]]' -e '^$@$$' | xargs

clean:
	@echo remove $(OUTPUTFOLDER) folder
	@rm -rf $(OUTPUTFOLDER)
	@echo done

build:
	@echo build release
  cargo build 
	@echo done

docker-build:
	@echo build image
	docker build -t $(DOCKER_IMAGE) -f Dockerfile .
	@echo done

docker-push:
	@echo push image
	docker tag $(DOCKER_IMAGE) $(DOCKER_REGISTRY)/$(DOCKER_IMAGE):$(DOCKER_TAG)
	aws ecr get-login --no-include-email --region eu-central-1 --profile aeternity-sdk | sh
	docker push $(DOCKER_REGISTRY)/$(DOCKER_IMAGE):$(DOCKER_TAG)
	@echo done

deploy-k8s-testnet:
	@echo deploy k8s
	kubectl -n testnet set image deployment/$(DOCKER_IMAGE) $(DOCKER_IMAGE)=$(DOCKER_REGISTRY)/$(DOCKER_IMAGE):$(DOCKER_TAG)
	@echo deploy k8s done

debug-start:
	npm install && VUE_APP_EPOCH_URL='$(VUE_APP_EPOCH_URL)' npm run serve
