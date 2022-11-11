import torch
from einops import repeat, rearrange
from torch import nn


class FlatModel(nn.Module):

    def __init__(self, size: int) -> None:
        super().__init__()

        self.size = size

        self.backbone = nn.Sequential(
            nn.Linear(size**2 + 1, 512),
            nn.SiLU(inplace=True),
            nn.Linear(512, 512),
            nn.SiLU(inplace=True),
        )

        self.policy_value = nn.Sequential(
            nn.Linear(512, 512),
            nn.SiLU(inplace=True),
            nn.Linear(512, size**2 + 1)
        )

    def forward(self, x: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
        x = self.backbone(x)
        x = self.policy_value(x)
        x[:, -1] = torch.tanh(x[:, -1])
        return x

    def get_example_input_array(self) -> torch.Tensor:
        return torch.zeros(1, self.size**2 + 1)


class ConvModel(nn.Module):

    def __init__(self, size: int) -> None:
        super().__init__()

        self.size = size

        self.backbone = nn.Sequential(
            nn.Conv2d(in_channels=2, out_channels=64, kernel_size=3, padding=1),
            nn.SiLU(inplace=True),
            nn.Conv2d(in_channels=64, out_channels=128, kernel_size=3, padding=1),
            nn.SiLU(inplace=True),
            # nn.Conv2d(in_channels=128, out_channels=256, kernel_size=3),
            # nn.SiLU(inplace=True),
        )
        self.flat_shape = self.get_flat_shape()
        print("self.flat_shape", self.flat_shape)

        self.policy = nn.Sequential(
            nn.Linear(self.flat_shape, 256),
            nn.SiLU(inplace=True),
            nn.Linear(256, self.size**2)
        )

        self.value = nn.Sequential(
            nn.Linear(self.flat_shape, 1),
            nn.Tanh()
        )

    def forward(self, x: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
        turns = x[:, -1]
        turns = repeat(turns, "b -> b 1 w h", w=self.size, h=self.size)

        x = x[:, :-1]

        x = rearrange(x, "b (w h) -> b 1 w h", w=self.size, h=self.size)
        x = torch.cat((x, turns), dim=1)
        x = self.backbone(x)

        x = x.reshape(-1, self.flat_shape)

        policy = self.policy(x)
        value = self.value(x)

        return torch.cat((policy, value), dim=1)

    def get_flat_shape(self) -> int:
        x = torch.zeros(1, 2, self.size, self.size)
        out = self.backbone(x)
        return out.flatten().size(0)

    def get_example_input_array(self) -> torch.Tensor:
        return torch.zeros(1, self.size**2 + 1)


def main():
    x = torch.randn(8, 3, 10, 10)
    model = Model()
    policy, value = model(x)
    print(policy.shape)
    print(value.shape)


if __name__ == "__main__":
    main()
