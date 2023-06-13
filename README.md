# Lido Satellite

## Run tests

1. Go to neutron's folder and run `make start`, wait for chain to launch
2. Get back to lido-satellite's folder and run `make build`
3. Run `./integration_test.bash` and wait until it finishes

It is expected to print

```
[OK] Main wallet has lost 3000 uibcatom
[OK] Second wallet has earned 500 uibcatom

INTEGRATION TESTS SUCCEDED
```

If it doesn't, something is really wrong.
