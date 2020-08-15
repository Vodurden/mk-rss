resource "aws_api_gateway_rest_api" "mk_rss" {
  name        = "mk-rss"
  description = "RSS feed generator"
}

resource "aws_api_gateway_resource" "proxy" {
  rest_api_id = aws_api_gateway_rest_api.mk_rss.id
  parent_id   = aws_api_gateway_rest_api.mk_rss.root_resource_id
  path_part   = "{proxy+}"
}

resource "aws_api_gateway_method" "proxy" {
  rest_api_id      = aws_api_gateway_rest_api.mk_rss.id
  resource_id      = aws_api_gateway_resource.proxy.id
  http_method      = "ANY"
  authorization    = "NONE"
}

resource "aws_api_gateway_integration" "mk_rss_lambda" {
  rest_api_id = aws_api_gateway_rest_api.mk_rss.id
  resource_id = aws_api_gateway_method.proxy.resource_id
  http_method = aws_api_gateway_method.proxy.http_method

  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.mk_rss.invoke_arn
}

resource "aws_api_gateway_method" "proxy_root" {
  rest_api_id      = aws_api_gateway_rest_api.mk_rss.id
  resource_id      = aws_api_gateway_rest_api.mk_rss.root_resource_id
  http_method      = "ANY"
  authorization    = "NONE"
}

resource "aws_api_gateway_integration" "mk_rss_lambda_root" {
  rest_api_id = aws_api_gateway_rest_api.mk_rss.id
  resource_id = aws_api_gateway_method.proxy_root.resource_id
  http_method = aws_api_gateway_method.proxy_root.http_method

  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.mk_rss.invoke_arn
}


resource "aws_api_gateway_deployment" "mk_rss" {
  depends_on = [
    aws_api_gateway_integration.mk_rss_lambda,
    aws_api_gateway_integration.mk_rss_lambda_root,
  ]

  rest_api_id = aws_api_gateway_rest_api.mk_rss.id
  stage_name  = "feed"
}

output "base_url" {
  value = aws_api_gateway_deployment.mk_rss.invoke_url
}
