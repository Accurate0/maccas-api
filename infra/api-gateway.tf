data "aws_apigatewayv2_api" "this" {
  api_id = var.AWS_API_GATEWAY_ID
}

resource "aws_apigatewayv2_route" "this" {
  for_each = { for x in jsondecode(file("endpoints.json")) : "${x.method} ${x.url}" => x }

  api_id             = data.aws_apigatewayv2_api.this.id
  route_key          = "${each.value.method} ${each.value.url}"
  target             = "integrations/${var.AWS_INTEGRATION_ID}"
  authorizer_id      = try(each.value.disableAuthorization, false) == true ? null : var.AWS_AUTHORIZER_ID
  operation_name     = each.key
  authorization_type = try(each.value.disableAuthorization, false) == true ? "NONE" : "JWT"
}
