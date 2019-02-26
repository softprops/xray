build:
	@docker build -t xray .

# https://docs.aws.amazon.com/xray/latest/devguide/xray-daemon-configuration.html
# https://docs.aws.amazon.com/xray/latest/devguide/xray-daemon-local.html#xray-daemon-local-docker
# get traces with
# EPOCH=$(date +%s) AWS_PROFILE=dev aws xray get-trace-summaries --start-time $(($EPOCH-60)) --end-time $(($EPOCH))
run: build
	@docker run \
			--rm \
      --attach STDOUT \
      -v ~/.aws/:/root/.aws/:ro \
      -e AWS_REGION=us-east-1 \
      -e AWS_PROFILE=$(AWS_PROFILE) \
      --name xray-daemon \
      -p 2000:2000/udp \
      xray -o -l dev