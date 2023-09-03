data "aws_apigatewayv2_api" "this-dev" {
  api_id = var.API_GATEWAY_ID_DEV
}

resource "aws_apigatewayv2_route" "this-dev" {
  for_each = { for x in jsondecode(file("endpoints.json")) : "${x.method} ${x.url}" => x }

  api_id             = data.aws_apigatewayv2_api.this-dev.id
  route_key          = "${each.value.method} ${each.value.url}"
  target             = "integrations/${var.AWS_INTEGRATION_ID_DEV}"
  authorizer_id      = try(each.value.disableAuthorization, false) == true ? null : aws_apigatewayv2_authorizer.this-dev.id
  operation_name     = each.key
  authorization_type = try(each.value.disableAuthorization, false) == true ? "NONE" : "JWT"
}

resource "aws_apigatewayv2_authorizer" "this-dev" {
  api_id           = data.aws_apigatewayv2_api.this-dev.id
  authorizer_type  = "JWT"
  identity_sources = ["$request.header.Authorization"]
  name             = "maccas-jwt-dev"
  jwt_configuration {
    audience = [var.ADB2C_APPLICATION_ID]
    issuer   = "https://apib2clogin.b2clogin.com/tfp/b1f3a0a4-f4e2-4300-b952-88f3dc55ee9b/b2c_1_signin/v2.0/"
  }
}
