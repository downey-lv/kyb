# Propably the fastest Latvian business registry search.  
  
Socket based search avoids making a new request for each letter.

With server in Germany and client in Latvia socket gives ~0.2 sec results on screen vs ~0.7 sec with REST.

This 0.5 sec comes from REST starting the whole request journey from beginning but socket was connected already.

Uses Rust and SQLite. 

Downloads daily fresh data from Latvijas Republikas Uzņēmumu reģistrs Open Data portal.

-- 

Most common use case for this is building the company registration flow for your (Latvian) app.

There are documented cases below (no data for filials etc) for which the software cannot help you. 

For these cases use [Uzņēmumu reģistrs API](https://www.ur.gov.lv/en/get-information/free-of-charge-services/api-web-services/) or Lursoft or Firmas.lv.

# Search Latvian company or government institution
+ REST API and websocket support.
+ Search with company name.
+ searches with or without Latvian special characters.
+ case independent.
+ lists precise match first
+ finds name, reg code, vat number, city, address, zip
+ results limited to 10 results
+ Removes SIA, AS etc from beginning and end of query.
+ Removes ' and " from query.
- problem: does not have data on filials
- problem: also no data on self-employed persons (this is in separate db in VID)
- problem: is missing embassy-s.

# Validate Latvian member of board
- Send person name, personal id and company registration code.
- Receive confirmation if this person is a member of board.

# Update and logging
- Keeps a local ~ 60 mb db of companies, public institutions and members of board
- Updates itself every night from Latvian Open Data portal.
- Logs verification results and errors.
- Logs searches and results

# REST API company search:
```
curl -X POST http://127.0.0.1:10001/lv/company -H "Content-Type: application/json" -d '{"name": "Raimond fantastic "}'

Response:
[{
  "legal_form":"SIA",
  "name":"Raimond fantastic",
  "city":"Jūrmala",
  "address":"Fantastics prospekts 123",
  "zip":"LV-1234",
  "public_sector":"0",
  "reg_code":"40203572370",
  "vat":true,
  "vat_number":"LV40203572370"
}]

Add `"include_pvd": true` to include matching PVD registrations by company registration number:
```
curl -X POST http://127.0.0.1:10001/lv/company -H "Content-Type: application/json" -d '{"name": "Coca-Cola", "include_pvd": true}'
```

Companies with PVD matches include a `pvd` array:
```
"pvd":[{
  "pvd_code":"006901",
  "address":"Ulbrokas iela 40, Rīga, LV-1021",
  "object_name":"noliktava"
},{
  "pvd_code":"092172",
  "address":"Latgales iela 495, Rumbula, Stopiņu pag., Ropažu nov., LV-2121",
  "object_name":"vairumtirdzniecības uzņēmums (AS \"Via 3L Latvia\"; PVD Nr.092089)"
}]
```

Search "", get []

Errors are not returned, enpty array [] is returned instead.
```
# Websocket company search:
```
Sample ws frontend client is in /release/client.js

// the following assumes some websocket client is installed as "ws"
ws ws://127.0.0.1:10001/lv/air
{"name": "Raimond fantastic"}

Response:
{
  "result":[{
    "legal_form":"SIA",
    "name":"Raimond fantastic",
    "city":"Jūrmala",
    "address":"Fantastics prospekts 1",
    "zip":"LV-1234",
    "public_sector":"0",
    "reg_code":"40203572370",
    "vat":true,
    "vat_number":"LV40203572370"
  }],
  "error":null
  }
  
or
{"name": "asdfadsf"}
{"result":[],"error":null}

{name: "Raimond fantastic"}     // missing "" around "name"
{"result":null,"error":"Failed to deserialize message"}

or
asdf
{"result":null,"error":"Failed to deserialize message"}

```

# Considerations - Company search and VAT
+ Assumes reg code (40203572370) and VAT Number (LV40203572370) number part match.
+ Because of this, the system currently gives false negatives (does not indicate VAT when it should) in cases when company reg code and VAT number do not match. 
+ This is the case usually with bigger corporations and governmental institutions. 
+ We pledge to fix it on first request.
+ The reason why it is this way, is because the open data VAT registry has "VAT Number" and "Name" fields, but no "Registration code" field, so matching it with the name may require a bit of extra engineering, as there is the freedom to either write SIA in front or after your business name and some other quirks.
+ There are companies with a registration code 000703116, so we use string for reg code field.

# Try member board validation
```
All fields must be strings.

curl -X POST http://127.0.0.1:10001/lv/board \
     -H "Content-Type: application/json" \
     -d '{"name": "Bērziņš Jānis", "personal_code": "201292-*****", "reg_code": "40103235360"}'
```

# Response for member of board
```
200 - { valid: true, verfication_id: 5 }
417 - Expectation failed - {"error":"Not member of board. Error id: 1238"} - Either is not member of board or data was incomplete.
501 - "Server down"
Empty response - I have used .unwrap()
```

# Considerations for Members of board validation:
+ Only first part of personal_code is matched, therefore we have "201292-*****" in sample. Second part comes anonymised from Latvian Open Data portal.
+ Person name order is lastname + firstname.
+ Person name has to be exact match (Bērziņš Jānis, not Berzins Janis)
+ Reg code has to be exact match and 11 characters (cannot have LV in beginning).
+ Every role in the CSV is considered as a positive match (Also Liqvidators and empty)
+ If CSV entry has personal code, then personal code is matched.
+ If CSV entry does not have personal code in db (like foreign owners), then personal code is not used for validation.
- problem: does not have proxies.

# Sources
```
// Members of board:
https://dati.ur.gov.lv/officers/officers.csv

// All companies in Latvia:
https://dati.ur.gov.lv/register/register.csv

// All public institutions in Latvia
https://dati.ur.gov.lv/register/ppi_public_persons_institutions.csv

All VAT payers in Latvia:
https://data.gov.lv/dati/dataset/9a5eae1c-2438-48cf-854b-6a2c170f918f/resource/610910e9-e086-4c5b-a7ea-0a896a697672/download/pdb_pvnmaksataji_odata.csv

PVD UR open data:
https://pakalpojumi.pvd.gov.lv/lv/opendata_files/ipvd_object_opendata/download
```

# Config
```
/src/config.rs
here you can set server address + port and schedule db update time.
```

# Build
```
// curl https://sh.rustup.rs -sSf | sh
// sudo apt install build-essential
// apt install libssl-dev
// apt install pkg-config
// sudo apt-get install libsqlite3-dev
cargo build --release
```
# Mac, Win and Linux executables
Available /release/ folder

Fair warning - the releases are not notarized, so your computer will suggest you several times not to install it.  

In Mac, (if you have "allow only notarized software" turned on), then after tryign to run the executable you may need to go to Settings -> Privacy / Security and click "Open anyway".

# Download latest DB
This will download the latest data from company register Open Data portal.
```
./kyb install
// or
cargo run install
```

# Run
```
cargo run
Runs by default at 127.0.0.1:10001
```
The index.html + client.js in releases file should be able to connect to the server.

# Test
Some tests require db to be installed and some require server to be running.
```
cargo test
cargo watch -x "test"
cargo watch -x "test query -- --nocapture"
cargo watch -x "test test_two_janises -- --nocapture"
```

# DB
Uses Sqlite for db.
Uses rust dirs crate to generalt platform spcecic path
```
sqlite3 /home/www/.local/share/kyb/kyb.db
sqlite3 /Users/martin/Library/Application\ Support/kyb/kyb.db
```

# Deploy
```
sudo nano /etc/systemd/system/kyb.service

[Unit]
Description=KYB Server
After=network.target

[Service]
User=myusername
ExecStart=/home/myusername/kyb/target/release/kyb
Restart=on-failure

[Install]
WantedBy=multi-user.target
```
Then:
```
sudo systemctl enable kyb
sudo systemctl start kyb
```
Logs:
```
journalctl -u kyb.service
journalctl -u kyb.service -f

```
# Update and rebuild
Pull new code, build new executable and restart nginx service (requires password for sudo)
```
// chmod +x rebuild.sh
./rebuild.sh
```

# License

Licensed under either of
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.
