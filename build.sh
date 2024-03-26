#!/usr/bin/env bash

sudo systemctl start docker
sudo docker buildx build . -t mediaplayer

sudo docker login
sudo docker tag mediaplayer 0x7030676e31/mediaplayer:latest
sudo docker push 0x7030676e31/mediaplayer:latest
