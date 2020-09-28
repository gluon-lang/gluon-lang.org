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
    "cpu": 0,
    "memory": 128,
    "logConfiguration": {
      "logDriver": "awslogs",
      "options": {
        "awslogs-region" : "eu-central-1",
        "awslogs-group" : "${aws_cloudwatch_log_group.gluon-lang.name}",
        "awslogs-stream-prefix" : "gluon-lang"
      }
    }
  }
]
DEFINITION
}


resource "aws_ecs_task_definition" "gluon_lang" {
  family                   = "gluon-lang"
  task_role_arn            = aws_iam_role.ecs_task_role.arn
  execution_role_arn       = aws_iam_role.ecs_task_execution_role.arn
  cpu                      = "256"
  memory                   = "1024"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  container_definitions = local.container_definitions
}

resource "aws_ecs_service" "gluon-lang" {
  name            = "gluon-lang"
  cluster         = aws_ecs_cluster.gluon-lang.id
  task_definition = aws_ecs_task_definition.gluon_lang.arn
  launch_type     = "FARGATE"

  desired_count = 1

  deployment_maximum_percent         = 100
  deployment_minimum_healthy_percent = 0

  network_configuration {
     subnets = aws_subnet.gluon-lang.*.id
     assign_public_ip = "true"

     security_groups = [aws_security_group.gluon_lang.id]
  }
}

resource "aws_security_group" "gluon_lang" {
  name_prefix = "gluon-lang-"
  vpc_id = aws_vpc.aws-vpc.id

  ingress {
    from_port = 0
    to_port = 0
    protocol = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port = 0
    to_port = 0
    protocol = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  lifecycle {
    create_before_destroy = true
  }
}
