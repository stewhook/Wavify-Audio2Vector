# Wavify

Convert raw audio files into a vectorized audio visualization.

## Description

Select an audio file as your target, select the amount of packets you would like to split the audio into. A higher packet count will result in a higher-quality visualization, but takes longer to process. Output can be grouped, or unioned into a single object.

## Getting Started

### Dependencies

```
"devDependencies": {
   "@figma/plugin-typings": "^1.90.0",
   "cpy-cli": "^5.0.0",
   "esbuild": "^0.24.0",
   "npm-run-all": "^4.1.5",
   "rimraf": "^5.0.0"
}
```

### Executing program

```
fork -> run [npm run build] -> In Figma (Desktop) -> Plugins -> Development -> Import Plugin From Manifest -> Done
```

## Authors

stewhook (samir@abuznaid.com)

## Planned Updates
```diff
+Batch audio uploads (Done)
+Height & Width customization (Done)
Different analysis algorithms
Isolate analysis by frequency
```

## Version History
* 0.3
   * Support for processing multiple files at once
* 0.2
   * Added Height & Width Customization
   * Linked OSRepo in support tab on Figma

* 0.1
    * Initial Release

## License

This project is licensed under the MIT License - see the LICENSE.md file for details
