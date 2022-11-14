import torch
from einops import repeat, rearrange
from torch import nn


class DepthwiseSeparableConv(nn.Module):
    def __init__(self, in_channels, out_channels, kernel_size, padding=0, bias=False):
        super().__init__()
        self.depthwise = nn.Conv2d(in_channels, in_channels, kernel_size, padding=padding, bias=bias, groups=in_channels)
        self.pointwise= nn.Conv2d(in_channels, out_channels, kernel_size=1, padding=padding, bias=bias)

    def forward(self, x):
        out = self.depthwise(x)
        out = self.pointwise(out)
        return out


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


class ResidualBlock(nn.Module):

    def __init__(self, channels: int, kernel_size: int = 3, padding: int = 1, bias: bool = False) -> None:
        super().__init__()

        self.conv = nn.Conv2d(channels, channels, kernel_size, padding, bias=bias)
        self.bn = nn.BatchNorm2d(channels)
        self.activation = nn.SiLU(inplace=False)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        x = self.conv(x)
        x = self.bn(x)
        x = self.activation(x)
        return x


class ConvModel(nn.Module):

    def __init__(self, size: int) -> None:
        super().__init__()

        self.size = size

        # self.in_conv = nn.Sequential(
        #     nn.Conv2d(2, 128, kernel_size=3, padding=1, bias=False),
        #     nn.BatchNorm2d(128),
        #     nn.SiLU(inplace=True),
        # )

        # self.backbone = nn.Sequential(
        #     self.in_conv,
        #     *[ResidualBlock(128) for _ in range(3)
        # ])

        self.backbone = nn.Sequential(
            DepthwiseSeparableConv(in_channels=2, out_channels=64, kernel_size=3),
            nn.SiLU(inplace=True),
            DepthwiseSeparableConv(in_channels=64, out_channels=128, kernel_size=3),
            nn.SiLU(inplace=True),
            DepthwiseSeparableConv(in_channels=128, out_channels=256, kernel_size=3),
            nn.SiLU(inplace=True),
        )

        # self.backbone = nn.Sequentia(
        #     nn.Conv2d(in_channels=2, out_channels=64, kernel_size=3),
        #     nn.SiLU(inplace=True),
        #     nn.Conv2d(in_channels=64, out_channels=128, kernel_size=3),
        #     nn.SiLU(inplace=True),
        #     nn.Conv2d(in_channels=128, out_channels=256, kernel_size=3),
        #     nn.SiLU(inplace=True),
        # )

        self.flat_shape = self.get_flat_shape()
        print("self.flat_shape", self.flat_shape)

        self.policy = nn.Sequential(
            nn.Linear(self.flat_shape, 256),
            nn.SiLU(inplace=True),
            nn.Linear(256, self.size**2)
        )

        self.value = nn.Sequential(
            nn.Linear(self.flat_shape, 128),
            nn.SiLU(inplace=True),
            nn.Linear(128, 1),
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
