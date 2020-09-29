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


resource "aws_ecs_task_definition" "gluon_lang" {
  family                   = "gluon-lang"
  task_role_arn            = aws_iam_role.ecs_task_role.arn
  execution_role_arn       = aws_iam_role.ecs_task_execution_role.arn
  memory                   = "256"
  network_mode             = "awsvpc"
  requires_compatibilities = ["EC2"]
  container_definitions = local.container_definitions
}

resource "aws_ecs_service" "gluon-lang" {
  name            = "gluon-lang"
  cluster         = aws_ecs_cluster.gluon-lang.id
  task_definition = aws_ecs_task_definition.gluon_lang.arn
  launch_type     = "EC2"

  desired_count = 1

  deployment_maximum_percent         = 100
  deployment_minimum_healthy_percent = 0

  network_configuration {
     subnets = aws_subnet.gluon-lang.*.id

     security_groups = [aws_security_group.gluon-lang.id]
  }
}

resource "aws_security_group" "gluon-lang" {
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


data "aws_ami" "ecs_optimized" {
  most_recent = true
  owners = ["591542846629"] # AWS

  filter {
      name   = "name"
      values = ["*amazon-ecs-optimized"]
  }

  filter {
      name   = "virtualization-type"
      values = ["hvm"]
  }
}

resource "aws_launch_configuration" "launch" {
  name          = "web_config"
  image_id      = data.aws_ami.ecs_optimized.id
  security_groups = [aws_security_group.gluon-lang.id]
  instance_type = "t2.micro"
}

data "template_file" "user_data" {
  template = "${file("${path.module}/user_data.yaml")}"

  vars = {
    ecs_cluster = "gluon-lang"
  }
}

resource "aws_instance" "gluon-lang" {
  ami                    = data.aws_ami.ecs_optimized.id
  subnet_id              = aws_subnet.gluon-lang.id
  instance_type          = "t2.nano"
  vpc_security_group_ids = [aws_security_group.gluon-lang.id]
  ebs_optimized          = "false"
  source_dest_check      = "false"
  associate_public_ip_address = "true"
  iam_instance_profile = aws_iam_instance_profile.ecs_agent.name
  user_data = data.template_file.user_data.rendered
  key_name = "home desktop"
}

resource "aws_iam_instance_profile" "ecs_agent" {
  name = "ecs-agent"
  role = aws_iam_role.ecs_agent.name
}
