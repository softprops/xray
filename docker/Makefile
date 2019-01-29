xray-build:
	@docker build -t xray .

xray-run: xray-build
	@docker run \
			--rm \
      --attach STDOUT \
      -v ~/.aws/:/root/.aws/:ro \
      --net=host \
      -e AWS_REGION=us-east-2 \
      --name xray-daemon \
      -p 2000:2000/udp \
      xray -o