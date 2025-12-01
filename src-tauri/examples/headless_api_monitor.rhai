// Target: HTTPBin (returns your IP)
let response = http.get("https://httpbin.org/ip");
let json = parse_json(response.body);

print(`Current Public IP: ${json.origin}`);

if json.origin.contains("127.0.0.1") {
     throw "Error: Proxy not working!";
}
return json;
