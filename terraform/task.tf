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
   name = "serverless_example_lambda"

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
