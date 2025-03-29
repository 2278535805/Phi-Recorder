from PIL import Image
from tkinter.filedialog import askopenfilename
from tkinter import Tk

_size = {"small": 16, "medium": 32, "big": 64}

file_path = askopenfilename()

def get_pixels(image, size):
    img = Image.open(file_path)
    img = img.resize((size, size)).convert("RGBA")

    pixels = []
    for i in range(img.height):
        for j in range(img.width):
            for x in img.getpixel((j, i)):
                pixels.append(x)
    return pixels


with open("icon_small.txt", "w") as f:
    f.write(f"{get_pixels(file_path, _size['small'])};")
with open("icon_medium.txt", "w") as f:
    f.write(f"{get_pixels(file_path, _size['medium'])};")
with open("icon_big.txt", "w") as f:
    f.write(f"{get_pixels(file_path, _size['big'])};")