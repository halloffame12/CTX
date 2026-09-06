class Outer:
    class Inner:
        def method(self):
            return 1

    def __init__(self):
        self.inner = Outer.Inner()

    @property
    def value(self):
        return self._value

    @value.setter
    def value(self, v):
        self._value = v

def module_level():
    x = 1
    return x
