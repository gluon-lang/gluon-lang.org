resource "aws_cloudwatch_log_group" "gluon-lang" {
  name = "gluon-lang"
  retention_in_days = "7"
}

locals {
  container_definitions = <<DEFINITION
[
  {
    "image": "marwes/try_gluon",
    "name": "gluon-lang",
    "logConfiguration": {
      "logDriver": "awslogs",
      "options": {
        "awslogs-region" : "eu-central-1",
        "awslogs-group" : "${aws_cloudwatch_log_group.gluon-lang.name}",
        "awslogs-stream-prefix" : "gluon-lang"
      }
    },
    "portMappings": [
        {
            "containerPort": 80
        },
        {
            "containerPort": 443
        }
    ]
  }
]
DEFINITION
}

resource "aws_apigatewayv2_api" "gluon-lang" {
    name = "gluon-lang"
    protocol_type = "HTTP"
}

resource "aws_apigatewayv2_api_mapping" "gluon-lang" {
    api_id = aws_apigatewayv2_api.gluon-lang.id
    domain_name = aws_apigatewayv2_domain_name.gluon-lang.id
    stage = "$default"
}

resource "aws_s3_bucket" "gluon-lang-doc" {
    bucket = "gluon-lang-doc"
    acl = "public-read"
    website {
      index_document = "index.html"
      error_document = "404.html"
    }
}

resource "aws_s3_bucket_policy" "gluon-lang-doc" {
  bucket = aws_s3_bucket.gluon-lang-doc.id
  policy = data.aws_iam_policy_document.site_public_access.json
}

data "aws_iam_policy_document" "site_public_access" {
  statement {
    actions = ["s3:GetObject"]
    resources = ["${aws_s3_bucket.gluon-lang-doc.arn}/*"]

    principals {
      type = "AWS"
      identifiers = ["*"]
    }
  }

  statement {
    actions = ["s3:ListBucket"]
    resources = ["${aws_s3_bucket.gluon-lang-doc.arn}"]

    principals {
      type = "AWS"
      identifiers = ["*"]
    }
  }
}

resource "null_resource" "remove_and_upload_to_s3" {
    provisioner "local-exec" {
      command = "aws s3 sync ../target/doc s3://${aws_s3_bucket.gluon-lang-doc.id}"
    }
}

resource "aws_iam_policy" "gluon-lang-doc" {
  name        = "gluon-lang-doc"
  path        = "/"
  description = "IAM policy for accessing documentation"

  policy = <<EOF
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "s3:Get*",
                "s3:List*"
            ],
            "Resource": "*"
        }
    ]
}
EOF
}


resource "aws_apigatewayv2_route" "gluon-lang" {
    api_id = aws_apigatewayv2_api.gluon-lang.id
    route_key = "$default"
    target = "integrations/${aws_apigatewayv2_integration.gluon-lang.id}"

    depends_on = [aws_apigatewayv2_integration.gluon-lang]
}

resource "aws_apigatewayv2_integration" "gluon-lang" {
    api_id = aws_apigatewayv2_api.gluon-lang.id
    integration_type = "AWS_PROXY"

    connection_type = "INTERNET"
    integration_method = "POST"
    integration_uri = aws_lambda_function.gluon-lang.invoke_arn

    payload_format_version = "2.0"

    lifecycle {
      create_before_destroy = true
    }
}

resource "aws_apigatewayv2_route" "gluon-lang-doc" {
    api_id = aws_apigatewayv2_api.gluon-lang.id
    route_key = "GET /doc/{proxy+}"
    target = "integrations/${aws_apigatewayv2_integration.gluon-lang-doc.id}"

    depends_on = [aws_apigatewayv2_integration.gluon-lang-doc]
}

resource "aws_apigatewayv2_integration" "gluon-lang-doc" {
    api_id = aws_apigatewayv2_api.gluon-lang.id

    integration_type = "HTTP_PROXY"
    integration_method = "GET"
    integration_uri = "https://gluon-lang-doc.s3.us-east-1.amazonaws.com/{proxy}"
}

resource "aws_apigatewayv2_domain_name" "gluon-lang" {
 domain_name = local.domain_name

  domain_name_configuration {
    certificate_arn = aws_acm_certificate.cert.arn
    endpoint_type   = "REGIONAL"
    security_policy = "TLS_1_2"
  }
}

resource "aws_lambda_permission" "apigw" {
   statement_id  = "AllowAPIGatewayInvoke"
   action        = "lambda:InvokeFunction"
   function_name = aws_lambda_function.gluon-lang.function_name
   principal     = "apigateway.amazonaws.com"

   # The "/*/*" portion grants access from any method on any resource
   # within the API Gateway REST API.
   source_arn = "${aws_apigatewayv2_api.gluon-lang.execution_arn}/*/*"
}

output "base_url" {
  value = aws_apigatewayv2_api.gluon-lang.api_endpoint
}

resource "aws_lambda_function" "gluon-lang" {
    function_name = "gluon-lang"

    filename = "../target/lambda.zip"
    source_code_hash = filebase64sha256("../target/lambda.zip")

    # "main" is the filename within the zip file (main.js) and "handler"
    # is the name of the property under which the handler function was
    # exported in that file.
    runtime = "provided.al2"
    handler = "main"

    role = aws_iam_role.lambda_exec.arn
}

# IAM role which dictates what other AWS services the Lambda function
# may access.
resource "aws_iam_role" "lambda_exec" {
   name = "gluon_lang_lambda"

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

resource "aws_iam_policy" "lambda_logging" {
  name        = "lambda_logging"
  path        = "/"
  description = "IAM policy for logging from a lambda"

  policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Action": [
        "logs:CreateLogGroup",
        "logs:CreateLogStream",
        "logs:PutLogEvents"
      ],
      "Resource": "arn:aws:logs:*:*:*",
      "Effect": "Allow"
    }
  ]
}
EOF
}

resource "aws_iam_role_policy_attachment" "lambda_logs" {
  role       = aws_iam_role.lambda_exec.name
  policy_arn = aws_iam_policy.lambda_logging.arn
}

resource "aws_acm_certificate" "cert" {
  domain_name       = local.domain_name
  validation_method = "DNS"
}

locals {
   domain_validation_option = tolist(aws_acm_certificate.cert.domain_validation_options).0
   domain_name = "gluon-lang.org"
}

resource "aws_route53_record" "cert_validation" {
  name    = local.domain_validation_option.resource_record_name
  type    = local.domain_validation_option.resource_record_type
  zone_id = local.zone_id
  records = [local.domain_validation_option.resource_record_value]
  ttl     = 60
}

resource "aws_acm_certificate_validation" "cert" {
  certificate_arn         = aws_acm_certificate.cert.arn
  validation_record_fqdns = [aws_route53_record.cert_validation.fqdn]
}
