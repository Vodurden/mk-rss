# This is a fake lambda function file so we can deploy our lambda code
# independently of terraform
data "archive_file" "dummy" {
  type        = "zip"
  output_path = "${path.module}/lambda_function_payload.zip"

  source {
    content = "not a real lambda, please upload the real lambda"
    filename = "dummy.txt"
  }
}

provider "aws" {
  region = "ap-southeast-2"
  allowed_account_ids = ["126498475487"]
}

resource "aws_lambda_function" "mk_rss" {
  function_name = "mk-rss"
  filename = data.archive_file.dummy.output_path

  # "main" is the filename within the zip file (main.js) and "handler"
  # is the name of the property under which the handler function was
  # exported in that file.
  # handler = "net.jakewoods.mkrss.lambda.APIGatewayHandler::handleRequest"
  handler = "doesnt.matter" # No handler needed for runtime `provided`
  runtime = "provided"

  memory_size = "128"
  timeout = "15"

  role = aws_iam_role.mk_rss.arn
}

resource "aws_iam_role" "mk_rss" {
  name = "mk_rss_lambda_role"

  assume_role_policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Action": "sts:AssumeRole",
      "Principal": {
        "Service": "lambda.amazonaws.com"
      },
      "Effect": "Allow",
      "Sid": ""
    }
  ]
}
EOF

}

resource "aws_iam_role_policy_attachment" "lambda_basic_execution_policy" {
  role       = aws_iam_role.mk_rss.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_lambda_permission" "api_gateway" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.mk_rss.function_name
  principal     = "apigateway.amazonaws.com"

  # The "/*/*" portion grants access from any method on any resource
  # within the API Gateway REST API.
  source_arn = "${aws_api_gateway_rest_api.mk_rss.execution_arn}/*/*"
}
