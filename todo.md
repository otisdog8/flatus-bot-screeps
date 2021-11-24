# a

11/22
Make perflogging
11/23
Make perflogging toggleable/stop after sufficient sample size.
Maybe update schema to support much higher res perf logs
FIGURE OUT PROCESSES AND SERIALIZATION. RUN ONE PROCESS

1. Implement scheduler - versioning included
   a. Process storage
   b. Preprocessing (pretick)
   c. Process execution
   d. Shifting
   e. Ending logic
   f. Process spawning

2. Implement kernel - versioning included

3. Implement IPC stuff

4. Implement spawning - remember spawning a creep should get own process
