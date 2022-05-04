from hikari import Vec3

class Script:
    def get_component(self):
        print("Lauranum")

class Translate(Script):
    speed = Vec3(0.5, 0.5, 0.5)
    def __init__(self, x, y):
        self.cur_pos = Vec3(x, y, 0.0)
        self.get_component()
    def update(self):
        dt = 0.3
        self.cur_pos += Translate.speed * dt * 0.5
