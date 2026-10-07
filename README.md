# How to Use

start listening port 3000
```bash
$ cargo run --release
```


choose an endpoint to request
```
http://<your_ip>/api/login
http://<your_ip>/api/register
```

whith a JSON body like:
```json
{
  "username": "my_name",
  "password": "my_pass"
}
```

# Observations
This is an academic exercise, not a professional project.
It does not use TLS for HTTPS, so it is not secure during the requests travel.
