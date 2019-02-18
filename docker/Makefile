build:
	@docker build -t xray .

# https://docs.aws.amazon.com/xray/latest/devguide/xray-daemon-configuration.html
# https://docs.aws.amazon.com/xray/latest/devguide/xray-daemon-local.html#xray-daemon-local-docker
run: build
	@docker run \
			--rm \
      --attach STDOUT \
      -v ~/.aws/:/root/.aws/:ro \
      -e AWS_REGION=us-east-2 \
      -e AWS_PROFILE=$(AWS_PROFILE) \
      --name xray-daemon \
      -p 2000:2000/udp \
      xray -o -l dev