# data "aws_apigatewayv2_api" "example" {
#   api_id = var.API_GATEWAY_ID
# }

# resource "aws_apigatewayv2_route" "this" {
#   for_each = { for x in jsondecode(file("endpoints.json")) : "${x.method} ${x.url}" => x }

#   api_id             = aws_apigatewayv2_api.this.id
#   route_key          = "${each.value.method} ${each.value.url}"
#   target             = "integrations/${aws_apigatewayv2_integration.this.id}"
#   authorizer_id      = try(each.value.disableAuthorization, false) == true ? null : aws_apigatewayv2_authorizer.this.id
#   operation_name     = each.key
#   authorization_type = try(each.value.disableAuthorization, false) == true ? "NONE" : "JWT"
# }
