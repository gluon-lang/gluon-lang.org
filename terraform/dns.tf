resource "aws_route53_record"  "gluon-lang" {
    zone_id = "Z39RMJR4RSLQT5"
    name = "gluon-lang.org"
    type = "A"
    ttl = 300

    records = [aws_instance.gluon-lang.public_ip]
}
