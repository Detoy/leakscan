# LeakScan threat model

## Who we protect
Creators, journalists, sources, activists, survivors of abuse and anyone who posts audio or video and doesn't want it used to work out where or when it was recorded.

## Adversary
Someone with a published clip, public flight-tracking archives, mains-frequency (ENF) reference data and time to spend. They look for:
1. **Aircraft Doppler passes.** The timing, the gaps between passes and the curve shape can be compared against known flights.
2. **Mains hum (50/60 Hz).** This shows the grid region. Tiny frequency drifts can be matched against ENF records to date a recording.
3. **Metadata.** Container tags, device info and creation times.

## What LeakScan does
- Detects (1) and (2) on the device and explains them.
- Attenuates them by about 40 dB with notches that follow each curve, removes hum and its harmonics, adds light masking noise, suggests cuts that remove the most identifying windows, then re-scans.
- Exports a WAV with no metadata, which covers (3) for the audio you export.

## Limits: it reduces exposure, it does not guarantee safety
- Very faint, broadband or overlapping cues can be missed. A score of 0 means "nothing found", not "safe".
- Notching leaves gaps that a careful analyst may still notice. Cues outside the analysed band (60–2000 Hz) are not handled.
- Cuts are suggestions. Leaving the identifying window in keeps most of the timing information.
- Other cues aren't covered yet: speech, background announcements, church bells, traffic, birdsong, room acoustics and visual content.
- Re-uploads, the original file, cloud backups and the post's own timestamp can all leak the same information.
- Video export isn't supported yet, so the original video's audio track and metadata are left untouched.

## Design rules
- No network use beyond loading the app's own static files. No uploads, accounts, analytics or third-party scripts.
- No geolocation, no flight-data matching, no maps or coordinates. Contributions that add any of these will be declined.
- All demo data is synthetic.

## Reporting
Please open a GitHub issue. For sensitive reports, use a private security advisory on the repository.
