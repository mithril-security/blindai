# BlindaiV2 end-to-end tests

- Split the terminal, on one tab, at the root of the project, run ```just run --release```. On the other, in the client directory run ```poetry shell``` and then ```poetry install```.

- In the client tab, run the setup script of the model you want to try to create the onnx and npz files. You can then run the tests of your choice.

If you need to, you can directly change the client's code and the changes will take effect, as if you installed with ```pip -e```.