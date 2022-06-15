# boost-farming Contract
A general farm, support:  
- multi-farm,
- lock seed to gain extra reward,
- global booster

### API
check `contracts/boost-farming/api.md`
### Compile
```bash
# to build locally, run
make
# to test the build, run
make test
# to test in sandbox
make sandbox
# to build stable release using docker, run
make release
# to clean, run
make clean
```
### Verify wasm
```bash
# to verify the wasm is the one on the chain, just run
./codehash.sh
# to verify the release you build equals the one in releases dir, just run
make release && ./codehash.sh
```