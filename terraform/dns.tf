locals {
    zone_id = "Z39RMJR4RSLQT5"
}

resource "aws_route53_record"  "gluon-lang" {
    zone_id = local.zone_id
    name = local.domain_name
    type = "A"

    alias {
        evaluate_target_health = true
        name                   = aws_apigatewayv2_domain_name.gluon-lang.domain_name_configuration.0.target_domain_name
        zone_id                = aws_apigatewayv2_domain_name.gluon-lang.domain_name_configuration.0.hosted_zone_id
    }
}
